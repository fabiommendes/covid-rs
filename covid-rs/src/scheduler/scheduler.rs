use crate::prelude::Time;
use std::{collections::VecDeque};

type Task<Ctx> = Box<dyn FnMut(&mut Ctx)>;

/// Enumeration of possible scheduling strategies
#[derive(Debug, Clone)]
pub enum Program {
    /// Execute task in the next invocation of before_step() or after_step()
    Now,

    /// Execute task in the beginning of the next step. Usually during the
    /// before_step() call.
    NextStep,

    /// Execute task every turn in the beginning each step.
    BeforeEveryStep,

    /// Execute task every turn in the end each step.
    AfterEveryStep,
 
    /// BeforeEvery(m, n) executes task in the beginning of every m steps
    /// with an initial delay of n turns.
    BeforeEvery(Time, Time),

    /// Similar to BeforeEvery(m, n), but executes in the end of the step.
    AfterEvery(Time, Time),

    /// Execute once after the given period in turns. An period of zero
    /// schedule task for the next step. Of one, waits one step and excutes
    /// in the next and so one.
    BeforeDelay(Time),

    /// Similar to BeforeInterval, but executes in the start of each step.
    AfterDelay(Time),
}

/// A simple turn-based scheduler that run tasks at specific discrete points in time.
pub struct Scheduler<Ctx> {
    immediate: Vec<Task<Ctx>>,
    before: SimpleScheduler<Ctx>,
    after: SimpleScheduler<Ctx>,
}

impl<Ctx> Scheduler<Ctx> {
    /// Create a new empty scheduler
    pub fn new() -> Self {
        return Scheduler {
            immediate: Vec::new(),
            before: SimpleScheduler::new(),
            after: SimpleScheduler::new(),
        };
    }

    fn _run_pending(&mut self, ctx: &mut Ctx) {
        for task in &mut self.immediate {
            task(ctx);
        }
        self.immediate.clear();
    }

    /// Advance scheduler by a single step and run all tasks scheduled to run in the
    /// beginning of the given step.
    pub fn before_step<'a>(&'a mut self, ctx: &'a mut Ctx) -> &mut Self {
        self._run_pending(ctx);

        while self.after.time < self.before.time {
            self.after.step(ctx);
        }

        self.before.step(ctx);
        return self;
    }

    /// Finish all tasks scheduled to run at the end of the current step.
    ///
    /// If before_step() was not called since the last call to after_steo(), run it before
    /// executing the scheduled tasks.
    ///
    /// In a typical situation, applications should implement the main loop wrapping all
    /// actions between before_step() and after_step() calls, like so
    ///
    /// ```rust
    /// fn main_loop() {
    ///     while executing() {
    ///         scheduler.before_step();
    ///         ... // loop actions
    ///         scheduler.after_step();
    ///     }
    /// }
    /// ```
    ///
    /// If application does not distinguish preparation from finalization actions, it should
    /// use only this method.  
    pub fn after_step(&mut self, ctx: &mut Ctx) -> &mut Self {
        self._run_pending(ctx);

        while self.before.time <= self.after.time {
            self.before.step(ctx);
        }

        self.after.step(ctx);
        return self;
    }

    /// Schedule task to run in the time frame specified by the given program.
    pub fn schedule_task(&mut self, when: Program, task: Task<Ctx>) {
        match when {
            Program::Now => self.immediate.push(task),
            Program::NextStep => self.before.schedule_once(task),
            Program::BeforeEveryStep => self.before.schedule_always(task),
            Program::AfterEveryStep => self.after.schedule_always(task),
            Program::BeforeEvery(period, delay) => self.before.schedule(delay, period, task),
            Program::AfterEvery(period, delay) => self.after.schedule(delay, period, task),
            Program::BeforeDelay(delay) => self.before.schedule(delay, 0, task),
            Program::AfterDelay(delay) => self.after.schedule(delay, 0, task),
        }
    }

    /// Schedule task to run in the time frame specified by the given program.
    pub fn schedule_function(&mut self, when: Program, f: impl FnMut(&mut Ctx) + 'static) {
        self.schedule_task(when, Box::new(f));
    }
}

impl<Ctx> Clone for Scheduler<Ctx> {
    fn clone(&self) -> Self {
        Self {
            immediate: self
                .immediate
                .iter()
                .map(try_clone)
                .filter_map(identity)
                .collect(),
            before: self.before.clone(),
            after: self.after.clone(),
        }
    }
}

/** SIMPLE SCHEDULER *********************************************************/

/// An internal struct used to reuse code for the main scheduler.
///
/// Implements basic scheduling tasks, but do not distinguishes the pre/after
/// step phases
// #[derive(Clone)]
struct SimpleScheduler<Ctx> {
    time: Time,
    once: Vec<Task<Ctx>>,
    always: Vec<Task<Ctx>>,
    schedule: VecDeque<(Time, Time, Task<Ctx>)>,
}

impl<Ctx> SimpleScheduler<Ctx> {
    pub fn new() -> Self {
        return SimpleScheduler {
            time: 0,
            once: Vec::new(),
            always: Vec::new(),
            schedule: VecDeque::new(),
        };
    }

    /// Add task to scheduler.
    ///
    /// Scheduler always keeps tasks sorted by execution time.
    pub fn schedule(&mut self, delay: Time, period: Time, task: Task<Ctx>) {
        let mut index = 0;
        let deadline = self.time + delay;
        for (i, (target, _, _)) in self.schedule.iter().enumerate() {
            index = i;
            if *target > deadline {
                break;
            }
        }
        self.schedule.insert(index, (deadline, period, task));
    }

    /// Add task to execute once.
    pub fn schedule_once(&mut self, task: Task<Ctx>) {
        self.once.push(task);
    }

    /// Add task to execute always.
    pub fn schedule_always(&mut self, task: Task<Ctx>) {
        self.always.push(task);
    }

    /// Run all tasks and update the time counter.
    pub fn step(&mut self, ctx: &mut Ctx) {
        // Run once
        for task in &mut self.once {
            task(ctx);
        }
        self.once.clear();

        // Run scheduled
        let time = self.time;
        loop {
            if let Some((deadline, _, _)) = self.schedule.get(0) {
                if *deadline <= time {
                    let (deadline, period, mut task) = self.schedule.pop_front().unwrap();
                    task(ctx);

                    if period > 0 {
                        self.schedule(deadline + period - time, period, task);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Run always
        for task in &mut self.always {
            task(ctx);
        }

        self.time += 1;
    }
}

impl<Ctx> Clone for SimpleScheduler<Ctx> {
    fn clone(&self) -> Self {
        let once = self
            .once
            .iter()
            .map(try_clone)
            .filter_map(identity)
            .collect();
        let always = self
            .always
            .iter()
            .map(try_clone)
            .filter_map(identity)
            .collect();
        let schedule = self
            .schedule
            .iter()
            .map(|(m, n, task)| try_clone(task).map(|x| (*m, *n, x)))
            .filter_map(identity)
            .collect();
        Self {
            time: self.time,
            once,
            always,
            schedule,
        }
    }
}

fn identity<T>(x: T) -> T {
    return x;
}

fn try_clone<Ctx>(_task: &Task<Ctx>) -> Option<Task<Ctx>> {
    // if let Some(f) = <dyn Any>::downcast_ref::<fn(&mut Ctx)>(task) {
    //     return Some(Box::new(f))
    // }
    return None;
}

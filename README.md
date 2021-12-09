Covid-rs
========

The covid-rs crate provides a Rust library with (preliminary) Python bindings and an executable to configure/execute agent-based epidemiological simulations.


## Executable

Pre-compiled versions of the executable are available and can be used to run simple epidemiological scenarios. The default executable is very flexible and can be configured via an TOML configuration file. This section describes the main sections and options available in this configuration format.

### Basic options

```toml
name = "some-name"
```

Name of simulation run. If not given, uses the name of the toml file without extension. It can be overridden with a command line option. The executable saves all results in a sub-folder named as `<name>-<timestamp>/`. This preserves by default the results of all simulations and the timestamp prevents sucessive runs to delete the previous results. The name must be a valid folder/file name.


```toml
verbose = true | false
```

If true, run in verbose mode and print debug messages.


### Population size and age distribution

Population size is controlled by one of these options

```toml
pop_size         = ...      # an integer
age_distribution = [...]    # array of floats
```

If only pop_size is given, it assumes an standard age distribution. Alternatively, it can be given together with age_distribution to determine the number of individuals in each age bracket. If only age_distribution is given, the sum of all values must provide the total population size. The array `age_distribution` represents the relative population sizes for each 10 year bracket. It must have 9 elements representing the age brackets of 0-9, 10-19, up to 70-79, 80+.



### Running simulation


```toml
max_iter = 90  # An integer. Each iteration corresponds to a single day.
```




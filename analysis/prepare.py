import os
import mundi
import pandas as pd
from subprocess import run
from pydemic.diseases import disease
from pathlib import Path
from matplotlib import pyplot as plt
from typing import cast
from warnings import warn
import concurrent.futures
# from mundi.plugins.epidemic import covid19

caso_full_url = "https://data.brasil.io/dataset/covid19/caso_full.csv.gz"
covid19 = disease("covid-19")
PATH = Path(__file__).parent / "data"
CASES = os.path.expanduser("~/caso_full.csv.gz")
RELEASE = 'release'
EXECUTABLE = Path(__file__).parent.parent / 'target' / RELEASE / 'covid'
EXECUTABLE = Path(__file__).parent / 'covid'
WINDOW_SIZE = 14
TOML_TEMPLATE = """
prob_infection = 0.10
n_contacts = 3.5
num_iter = {num_iter}
verbose = false
pop_counts = {pop_counts}

# Info
delay = {delay}
attack_rate = {attack}

[epicurve]
data = {epicurve_data}
smoothness = {smoothness}
"""

def prepare_region(path: Path, region: mundi.Region):
    """
    Create files to initialize a state.
    """

    # Age distribution 
    df = region.age_distribution
    distrib = df.values.copy()[:18:2]
    distrib += df.values[1:18:2]
    distrib[-1] += df.values[18:].sum()
    
    # Estimate cases from deaths
    curve = covid19.epidemic_curve(region, path=CASES)
    deaths = cast(pd.Series,
        curve["deaths"]
        .rolling(WINDOW_SIZE, center=True, win_type="triang")
        .mean()
        .fillna(method="bfill")
        .dropna()
    )
    params = covid19.params(region=region)
    cases = (deaths / params.IFR).astype("int")
    epicurve = cases.diff().fillna(0).astype("int").values
    attack = 100 * cases.iloc[-1] / region.population
    print("Attack rate: {:n}%".format(attack))
    
    # Clean epicurve
    i, j = 0, len(epicurve) - 1
    while epicurve[i] == 0:
        i += 1
    
    while epicurve[j] == 0:
        j -= 1
    
    if (n := len(epicurve) - j -1):
        m = n + WINDOW_SIZE // 2
        epicurve = list(epicurve)[:j - WINDOW_SIZE // 2]
        print(f'WARNING: {region.id} tail with {n} null items. trucanting epicurve to a {m} delay')
        n += WINDOW_SIZE // 2
    epicurve = epicurve[i:j]
    
    # Create config
    conf = TOML_TEMPLATE.format(
        num_iter=60,
        pop_counts=list(distrib),
        epicurve_data=list(epicurve),
        smoothness=0.75,
        delay=n,
        attack=attack,
    )    
    
    with open(path / 'conf.toml', 'w') as fd:
        fd.write(conf)


def prepare_all(path: Path = PATH):
    """
    Prepare all states data
    """
    paths = []
    states = mundi.regions(type="state", country="BR")
    for state in sorted(states):
        print(f"\nprocessing {state}")
        data = path / state.id
        data.mkdir(parents=True, exist_ok=True)
        prepare_region(data, state)
        paths.append(data)

    with concurrent.futures.ThreadPoolExecutor() as executor:
        futures = []
        for path in paths:
            futures.append(executor.submit(run_simulation, path=path))


def run_simulation(path):
    print(f'running {path}')
    run(EXECUTABLE, cwd=path)
    res: pd.DataFrame = pd.read_csv(path / 'epicurve.csv')
    res['new_cases'] = -res['S'].diff().fillna(0).astype('int')
    res['new_deaths'] = res['D'].diff().fillna(0).astype('int')
    res['cases'] = res['S'].iloc[0] - res['S']
    res['deaths'] = res['D'] - res['D'].iloc[0]
    res.to_csv(path / 'epicurve.csv')
    print(f'analysis finished: {path}')
    

if __name__ == "__main__":
    import typer

    typer.run(prepare_all)
import datetime
import os
import mundi
import pandas as pd
from subprocess import run
from pydemic.diseases import disease
from pathlib import Path
from matplotlib import pyplot as plt
# from mundi.plugins.epidemic import covid19

covid19 = disease("covid-19")
PATH = Path(__file__).parent / "data"


def plot_region(path: Path, region: mundi.Region, ext='png'):
    """
    Plot state.
    """
    df = pd.read_csv(path / 'epicurve.csv').iloc[1:]
    plot_data(path, df, region, ext)
    df.index.name = 'day'
    return df


def plot_data(path, data: pd.DataFrame, region, ext='png', future=60, past=120):
    data = data.iloc[-(past + future):].copy().reset_index()
    data.index -= past
    today = datetime.datetime.now().date()
    
    def common(name):
        plt.legend()
        plt.tight_layout()
        _y0, y1 = plt.ylim()
        plt.plot([0, 0], [0, y1], 'k--', lw=2)
        plt.ylim(0, y1)
        plt.xlim(data.index[0], data.index[-1])
        plt.grid(True)
        # plt.show() 
        plt.savefig(path / name)
        plt.clf()

    deaths = data['new_deaths']
    deaths.rolling(14, 1, center=True, win_type='triang').mean().plot(label='média móvel')
    deaths.plot(label='mortes/dia')
    plt.xlabel(f'dias (a partir de {today})')
    plt.ylabel('mortes')
    plt.title(f'Projeção de óbitos por Covid-19 ({region.name})')
    common(f'obitos.{ext}')

    cases = data['new_cases']
    cases.rolling(14, 1, center=True, win_type='triang').mean().plot(label='média móvel')
    cases.plot(label='casos/dia')
    plt.xlabel(f'dias (a partir de {today})')
    plt.ylabel('casos')
    plt.title(f'Projeção de casos por Covid-19 ({region.name})')
    common(f'casos.{ext}')

    icu = data['C']
    icu.rolling(14, 1, center=True, win_type='triang').mean().plot(label='média móvel')
    icu.plot(label='leitos UTI')
    plt.xlabel(f'dias (a partir de {today})')
    plt.ylabel('leitos ocupados')
    plt.title(f'Projeção de pressão hospitalar por Covid-19 ({region.name})')
    common(f'criticos.{ext}')


def plot_all(path: Path = PATH):
    """
    Prepare all states data
    """
    states = mundi.regions(type="state", country="BR")
    data = []
    for r in sorted(states):
        print(f"\nprocessing {r}")
        data.append(plot_region(PATH / r.id, r).reset_index())
    
    br = path / "BR"
    br.mkdir(exist_ok=True)
    df = pd.concat(data).groupby('day').sum()
    plot_data(br, df, mundi.region('BR'))
    df.to_csv(br / 'epicurve.csv')

if __name__ == "__main__":
    import typer

    typer.run(plot_all)
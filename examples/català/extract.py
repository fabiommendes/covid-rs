import click
import pandas as pd
import datetime

@click.group()
def main():
    ...


@main.command("cases")
@click.argument("input", type=click.File())
def cases(input: click.File):
    df: pd.DataFrame = pd.read_csv(input)
    df.columns = map(str.lower, df.columns)
    df = df.rename(
        columns={
            "data": "date",
            "nous casos diaris confirmats": "new_cases",
            "defuncions diàries": "new_deaths",
            "total de casos confirmats": "cases",
            "total de defuncions": "deaths",
        }
    )
    df["date"] = [datetime.datetime.strptime(x, '%d/%m/%Y').date() for x in df["date"]]
    df = df[["date", "new_cases", "new_deaths"]]
    for col in ["new_cases", "new_deaths"]:
        df[col] = df[col].fillna(0).astype(int)
    df = df.set_index("date").sort_index()
    
    print(df)


@main.command("vaccine")
@click.argument("input", type=click.File())
def cases(input):
    print(input)


if __name__ == "__main__":
    main()

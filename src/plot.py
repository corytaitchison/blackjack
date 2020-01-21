import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
sns.set()

f = open("tmp/with_bet.txt")
balances = [int(x) for x in f.read().split(", ")]
df = pd.DataFrame(balances).melt()

f = open("tmp/without_bet.txt")
balances = [int(x) for x in f.read().split(", ")]
df["Without Strategy"] = balances[:2442854]

f = open("tmp/random_bet.txt")
balances = [int(x) for x in f.read().split(", ")]
df["Random"] = balances[:2442854]

df = df.drop(['variable'], axis=1).rename(
    columns={"value": "With Strategy"}).melt()
df['index'] = [x % 2442854 for x in df.index]

ax = sns.lineplot(x="index", y="value", hue="variable", data=df)
ax.set(xlabel="Hand", ylabel="Balance", title="")
plt.show()

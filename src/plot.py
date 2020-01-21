import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import seaborn as sns
sns.set()

f = open("tmp/basic_no_bet.txt")
balances = [int(x) for x in f.read().split(", ")]
df = pd.DataFrame(balances[:1461563]).melt()

f = open("tmp/no_strategy.txt")
balances = [int(x) for x in f.read().split(", ")]
df["Dealer Strategy"] = balances[:1461563]

f = open("tmp/random.txt")
balances = np.pad([int(x) for x in f.read().split(", ")],
                  (0, 1461563-142949), constant_values=0)
df["Random"] = balances[:1461563]

df = df.drop(['variable'], axis=1).rename(
    columns={"value": "Basic Strategy"}).melt()
df['index'] = [x % 1461563 for x in df.index]

ax = sns.lineplot(x="index", y="value", hue="variable", data=df)
ax.set(xlabel="Hand", ylabel="Balance",
       title="Blackjack Earnings With Various Strategies")
plt.show()

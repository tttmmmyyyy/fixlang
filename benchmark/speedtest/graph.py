import pandas as pd
import matplotlib.pyplot as plt

# CSVファイルの読み込み
df = pd.read_csv('log.csv')

# 最新の30件のデータを抽出
df = df.tail(30)
print(df)

# コミットハッシュ（最初の7文字）のリスト
commit_hashes = df['commit_hash'].str[:7]

# グラフを作成
fig, ax1 = plt.subplots(figsize=(10, 6))

# 左側の軸（instructions）のプロット
ax1.set_xlabel('Commit')
ax1.set_ylabel('Instructions', color='tab:blue')
ax1.plot(commit_hashes, df['instructions'],
         color='tab:blue', marker='o', label='Instructions')
ax1.tick_params(axis='y', labelcolor='tab:blue')

# 右側の軸（memory_accesses）のプロット
ax2 = ax1.twinx()
ax2.set_ylabel('Memory Accesses', color='tab:red')
ax2.plot(commit_hashes, df['memory_accesses'],
         color='tab:red', marker='o', label='Memory Accesses')
ax2.tick_params(axis='y', labelcolor='tab:red')

# グラフのタイトル
plt.title('Speedtest Log')

# グラフを表示
plt.xticks(rotation=45)
plt.tight_layout()

# 画像として保存
plt.savefig('graph.svg')

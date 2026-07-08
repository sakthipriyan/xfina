import pandas as pd
df = pd.read_excel("../xfina-test-data/bank-accounts/bob/raw/OpTransactionHistoryUX508-07-2026.xls", header=None)
print(df.head(20).to_string())

from functools import reduce
arr = list(range(1, 10))
ans = reduce(lambda acc, x: acc * x, arr, 1)
print(ans)
import math
LIM = 100
arr = []
for a in range(0, LIM):
    for b in range(0, LIM):
        arr.append(math.gcd(a, b))

print(' '.join([str(e) for e in arr]))
a = 0
while a != 1_000_000_000:
    if a * a % (1_000_000_009) == 56480:
        break
    a += 1
print(a)
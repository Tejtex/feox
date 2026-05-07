def spf(n):
    if n < 2: 
        return "nil"
    small_primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
    for p in small_primes:
        if n % p == 0:
            return p
    if n < 1369: 
        return n
    p = 41;
    while p * p <= n:
        if n % p == 0:
            return p 
        p = p + 2;
        
    return n
    
print(spf(1))
print(spf(5))
print(spf(12))
print(spf(45))
print(spf(67))
print(spf(99))
print(spf(998244353))
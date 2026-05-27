pub fn sieve(n: usize) -> Vec<usize> {
    if n < 2 {
        return Vec::new();
    }

    let mut prime = vec![true; n + 1];
    prime[0] = false;
    prime[1] = false;

    let mut p = 2;
    while p * p <= n {
        if prime[p] {
            let mut multiple = p * p;
            while multiple <= n {
                prime[multiple] = false;
                multiple += p;
            }
        }
        p += 1;
    }

    prime
        .into_iter()
        .enumerate()
        .filter_map(|(idx, is_prime)| is_prime.then_some(idx))
        .collect()
}

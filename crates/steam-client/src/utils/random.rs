pub(crate) fn random_u128() -> Result<u128, getrandom::Error> {
    let mut bytes = [0; 16];
    getrandom::fill(&mut bytes)?;
    Ok(u128::from_ne_bytes(bytes))
}

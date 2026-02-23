use anyhow::{Result, anyhow};

pub fn get_index(co_ordinates: &[usize], dimensions: &[usize]) -> Result<usize> {
    let dimensionality = dimensions.len();
    if co_ordinates.len() != dimensionality {
        return Err(anyhow!(
            "Sizes differ: {} != {}",
            co_ordinates.len(),
            dimensions.len()
        ));
    }
    let mut result = co_ordinates[0];
    if dimensionality > 1 {
        for offset in 0..dimensionality - 1 {
            result = result * dimensions[offset] + co_ordinates[offset + 1];
        }
    }
    Ok(result)
}

pub fn next_co_ordinates(co_ordinates: &mut [usize], dimensions: &[usize]) {
    let dimensionality = dimensions.len();
    let mut j = dimensionality - 1;
    loop {
        co_ordinates[j] += 1;
        if co_ordinates[j] < dimensions[j] {
            break;
        } else {
            co_ordinates[j] = 0;
            if j < 1 {
                break;
            }
            j -= 1;
        }
    }
}

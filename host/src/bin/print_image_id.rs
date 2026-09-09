use percolate_methods::PERCOLATE_GUEST_ID;
use risc0_zkvm::sha::Digest;

fn main() {
    let digest: Digest = PERCOLATE_GUEST_ID.into();
    println!("IMAGE_ID: {}", digest);
}

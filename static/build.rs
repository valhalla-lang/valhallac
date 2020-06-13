mod read_version;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	read_version::out()?;

	Ok(())
}

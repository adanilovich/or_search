build:
	@cargo build --release && cp ./target/release/or_search .

ut:
	@cargo test -- --nocapture

ut-single:
	@cargo test -- --nocapture $(test)

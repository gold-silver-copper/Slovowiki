DUMP ?= /Users/kisaczka/Desktop/code/english/raw-wiktextract-data.jsonl
OFFICIAL ?= data/official-isv.csv
DATA ?= data/wiktionary-lab.json
OUT ?= target/eval
PORT ?= 8765

.PHONY: extract-proto eval build serve explain check fmt test clean

# One-time: stream the 23GB dump into the Proto-Slavic cache (enables +proto-derived).
extract-proto:
	cargo run --release -- extract-proto --dump "$(DUMP)"

# Reproducible accuracy benchmark against the official Interslavic dictionary.
eval:
	cargo run --release -- evaluate --official "$(OFFICIAL)" --out "$(OUT)"

# Build the website dataset from the official dictionary's Slavic evidence.
build:
	cargo run --release -- build --official "$(OFFICIAL)" --output "$(DATA)"

serve:
	cargo run --release -- serve --data "$(DATA)" --port $(PORT)

# Spot-check one word/gloss with a full rule trace, e.g. `make explain W=duša`.
explain:
	cargo run -- explain "$(W)"

fmt:
	cargo fmt

check:
	cargo check

test:
	cargo test

clean:
	rm -f data/wiktionary-lab.json data/*.tmp

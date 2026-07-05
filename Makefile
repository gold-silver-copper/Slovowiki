DUMP ?= /Users/kisaczka/Desktop/code/english/raw-wiktextract-data.jsonl
DATA ?= data/wiktionary-lab.json
PORT ?= 8765

.PHONY: build full serve clean check

build:
	cargo run -- build --dump "$(DUMP)" --output "$(DATA)" --max-proto 200 --max-lexemes 10000

full:
	cargo run --release -- build --dump "$(DUMP)" --output "$(DATA)"

serve:
	cargo run -- serve --data "$(DATA)" --port $(PORT)

check:
	cargo check

clean:
	rm -f data/*.json data/*.tmp

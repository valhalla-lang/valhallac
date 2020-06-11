#!/bin/sh

cargo deps --all-deps --include-orphans --subgraph \
			lazy_static regex unindent snailquote unicode-width \
			enum-primitive-derive num-traits colored toml \
		--subgraph-name "Direct Dependencies" \
		--manifest-path "$PWD/Cargo.toml" \
	| dot -Tpng \
		-Nfontname='Iosevka Term' -Gfontname='Iosevka Term' \
		-Gsize='3,2\!' -Gratio=auto -Gdpi=1000 \
		-o"$PWD/graph.png"


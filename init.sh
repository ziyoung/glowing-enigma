init-container() {
  apt update
  apt install libjemalloc-dev
  apt install graphviz
}

run() {
  	export RUST_BACKTRACE=1
	  export _RJEM_MALLOC_CONF=prof:true,lg_prof_interval:28
	  cargo run
}

analysis-prof() {
  bin="./target/debug/memory-leak-example"
  for id in {0..1}
  do
    file="./profile/regex-caputres-iter-$id.prof"
    jeprof --svg "$bin" "$file" > "./profile/profile-$id.svg"
    jeprof --collapse "$bin" "$file" | perl flamegraph.pl > "./profile/svg/flame-$id.svg"
  done
}


# loaload_and_gen() {
#   curl http://127.0.0.1:3000/debug/profile/prof > "$1.base.prof"
# }

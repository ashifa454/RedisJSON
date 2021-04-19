
define HELP
make build
  DEBUG=1          # build debug variant
make clean         # remove binary files
  ALL=1            # remove binary directories

make all           # build all libraries and packages

make pytest        # run tests
  TEST=name        # run test matching 'name'
  TEST_ARGS="..."  # RLTest arguments
  GEN=0|1          # run general tests on a standalone Redis topology
  AOF=0|1          # run AOF persistency tests on a standalone Redis topology
  SLAVES=0|1       # run replication tests on standalone Redis topology
  CLUSTER=0|1      # run general tests on a OSS Redis Cluster topology
  VALGRIND|VD=1    # run specified tests with Valgrind

make package       # build package (RAMP file)

make docker
make docker_push

make builddocs
make localdocs
make deploydocs

endef

#----------------------------------------------------------------------------------------------

MODULE_NAME=rejson.so

ifeq ($(DEBUG),1)
TARGET_DIR=target/debug
else
CARGO_FLAGS += --release
TARGET_DIR=target/release
endif

TARGET=$(TARGET_DIR)/$(MODULE_NAME)

#----------------------------------------------------------------------------------------------

all: build

#----------------------------------------------------------------------------------------------

lint:
	cargo fmt -- --check

#----------------------------------------------------------------------------------------------

build:
	cargo build --all --all-targets $(CARGO_FLAGS)
	cp $(TARGET_DIR)/librejson.so $(TARGET)

clean:
ifneq ($(ALL),1)
	cargo clean
else
	rm -rf target
endif

#----------------------------------------------------------------------------------------------

test: pytest

pytest:
	MODULE=$(abspath $(TARGET)) ./tests/pytest/tests.sh

cargo_test:
	cargo test --features test --all

.PHONY: pytest cargo_test

#----------------------------------------------------------------------------------------------

BENCHMARK_ARGS = redisbench-admin run-local

ifneq ($(REMOTE),)
	BENCHMARK_ARGS = redisbench-admin run-remote
endif

BENCHMARK_ARGS += --module_path $(realpath $(TARGET))
ifneq ($(BENCHMARK),)
	BENCHMARK_ARGS += --test $(BENCHMARK)
endif


benchmark: $(TARGET)
	cd ./tests/benchmarks; $(BENCHMARK_ARGS) ; cd ../../

#----------------------------------------------------------------------------------------------

package:
	$(MAKE) -C ./src package

.PHONY: package

#----------------------------------------------------------------------------------------------

docker:
	docker build --pull -t rejson:latest .

docker_push:
	docker push redislabs/rejson:latest

.PHONY: docker docker_push

#----------------------------------------------------------------------------------------------

builddocs:
	mkdocs build

localdocs: builddocs
	mkdocs serve

deploydocs: builddocs
	mkdocs gh-deploy

.PHONY: builddocs localdocs deploydocs

#----------------------------------------------------------------------------------------------

ifneq ($(HELP),) 
ifneq ($(filter help,$(MAKECMDGOALS)),)
HELPFILE:=$(shell mktemp /tmp/make.help.XXXX)
endif
endif

help:
	$(file >$(HELPFILE),$(HELP))
	@echo
	@cat $(HELPFILE)
	@echo
	@-rm -f $(HELPFILE)

.PHONY: help

load("@bin_crates//:defs.bzl", bin_crates_repositories = "crate_repositories")
load("@lib_crates//:defs.bzl", lib_crates_repositories = "crate_repositories")

def build_deps_setup():
    bin_crates_repositories()
    lib_crates_repositories()

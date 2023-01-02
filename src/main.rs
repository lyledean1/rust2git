mod git;

// manually run export DYLD_LIBRARY_PATH=$(pwd)/libgit2-sys/target/source-v1.5.0/build
fn main() {
    let path = std::env::args().skip(1).next().expect("usage: git-toy PATH");
    let repo = git::Repository::open(&path).expect("opening repo");

    let commit_oid = repo.reference_name_to_id("HEAD")
        .expect("looking up head reference");

    let commit = repo.find_commit(&commit_oid).expect("looking up commit");

    let author = commit.author();

    println!("{} <{}>\n", author.name().unwrap_or("none"), author.email().unwrap_or("none"));

    println!("{}", commit.message().unwrap_or("none"));

}
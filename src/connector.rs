struct Config {}

enum Command {
    Spec,
    Check(Config),
    Discover(Config),
    Read(Config),
}

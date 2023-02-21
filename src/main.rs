use elf_parser::parser::ElfParser;

fn main() {
    let contents = std::fs::read("./out/rv64i-test").unwrap();
    let _ = ElfParser::parse(contents);
}

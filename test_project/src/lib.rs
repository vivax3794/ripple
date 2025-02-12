use ripple_runtime::component;

component!(hello);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn wip() {
        assert_eq!(hello::hello(), 100);
    }
}

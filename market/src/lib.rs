extern crate solana_program;
extern crate bytemuck;
extern crate safe_transmute;
extern crate arrayref;

pub mod raydium;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

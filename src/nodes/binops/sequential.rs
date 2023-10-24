/// This macro avoids even reading from a queue, whenever this queue was the last one being handled.
/// This assures that even in the case of multiple handles on the same input in absense of a single
/// handle of the other, all elements are preserved in a sequential order.
///
/// This is only important for BinOps that must satisfy a trait with three different generics of the
/// form: I1: BinOp<I2, Output=O> as those trait bounds won't allow a bin operation on e.g. I1 + I1.
#[macro_export]
macro_rules! handle_sequentially {
    ($lhs_input:ident, $rhs_input:ident, $lhs_handle:ident, $rhs_handle:ident) => {
        fn on_update(&mut self) -> anyhow::Result<(), UpdateError> {
            match self.state {
                super::binops::BinOpState::I1(_) => {
                    if let Ok(i2) = self.$rhs_input.next() {
                        self.$rhs_handle(i2)?;
                    }
                }
                _ => {
                    if let Ok(i1) = self.$lhs_input.next() {
                        self.$lhs_handle(i1)?;
                    }
                }
            }
            // The functionality is repeated to handle two inputs per epoche
            match self.state {
                super::binops::BinOpState::I1(_) => {
                    if let Ok(i2) = self.$rhs_input.next() {
                        self.$rhs_handle(i2)?;
                    }
                }
                _ => {
                    if let Ok(i1) = self.$lhs_input.next() {
                        self.$lhs_handle(i1)?;
                    }
                }
            }
            Ok(())
        }
    };
}

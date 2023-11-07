// use rand::Rng;
// use std::iter;

// // Define a custom type for u64 to implement required traits
// #[derive(Debug, Clone, Copy, PartialEq)]
// struct U64Type(u64);

// impl U64Type {
//     fn one() -> Self {
//         U64Type(1)
//     }

//     fn random(rng: &mut impl Rng) -> Self {
//         U64Type(rng.gen())
//     }

//     fn mul(self, rhs: Self) -> Self {
//         U64Type(self.0.wrapping_mul(rhs.0))
//     }
// }

// // Implement the Mul and Add traits for U64Type
// use std::ops::{Add, Mul, MulAssign};

// impl Mul for U64Type {
//     type Output = Self;

//     fn mul(self, rhs: Self) -> Self {
//         U64Type(self.0.wrapping_mul(rhs.0))
//     }
// }

// impl Add for U64Type {
//     type Output = Self;

//     fn add(self, rhs: Self) -> Self {
//         U64Type(self.0.wrapping_add(rhs.0))
//     }
// }

// impl MulAssign<U64Type> for U64Type {
//     fn mul_assign(&mut self, other: U64Type) {
//         self.0 *= other.0;
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::U64Type;
//     use rand::Rng;
//     use std::iter;

//     #[test]
//     fn test1() {
//         let blinding_factors = 5; // Modify this according to your needs
//         let params_n = 15; // Modify this according to your needs

//         let mut rng = rand::thread_rng();

//         let lookup_product: Vec<U64Type> = (0..params_n).map(|_| U64Type(rng.gen())).collect();

//         let z: Vec<U64Type> = iter::once(U64Type::one())
//             .chain(lookup_product.iter().cloned())
//             .scan(U64Type::one(), |state, cur| {
//                 *state = state.mul(cur);
//                 Some(*state)
//             })
//             .take(params_n - blinding_factors)
//             .chain((0..blinding_factors).map(|_| U64Type::random(&mut rng.clone())))
//             .collect();

//         println!("{:?}", z);
//         assert_eq!(z.len(), params_n);

//         fn compress_expressions(expressions: &[U64Type], theta: U64Type) -> U64Type {
//             expressions
//                 .iter()
//                 .cloned()
//                 .fold(U64Type::one(), |acc, expression| acc * theta + expression)
//         }

//         let theta = U64Type(2);

//         let permuted_input_expression = [
//             U64Type(1),
//             U64Type(2),
//             U64Type(3),
//             U64Type(4),
//             U64Type(5),
//             U64Type(6),
//             U64Type(7),
//             U64Type(8),
//             U64Type(9),
//             U64Type(10),
//         ];
//         let permuted_table_expression = [
//             U64Type(10),
//             U64Type(9),
//             U64Type(8),
//             U64Type(7),
//             U64Type(6),
//             U64Type(5),
//             U64Type(4),
//             U64Type(3),
//             U64Type(2),
//             U64Type(1),
//         ];

//         let compressed_input_expression = compress_expressions(&permuted_input_expression, theta);
//         let compressed_table_expression = compress_expressions(&permuted_table_expression, theta);
//         let beta = U64Type(1);
//         let gamma = U64Type(1);
//         let u = 10;

//         for i in 0..u {
//             let mut left = z[i + 1];
//             let permuted_input_value = permuted_input_expression[i];

//             let permuted_table_value = permuted_table_expression[i];

//             left *= beta + permuted_input_value;
//             left *= gamma + permuted_table_value;

//             let mut right = z[i];
//             let input_term = compressed_input_expression[i];
//             let table_term = compressed_table_expression[i];

//             let input_term_with_beta = input_term + beta;
//             let table_term_with_gamma = table_term + gamma;

//             right *= input_term_with_beta * table_term_with_gamma;

//             assert_eq!(left, right);
//         }
//     }

//     #[test]
//     fn test2() {
//         fn compress_expressions(expressions: &[U64Type], theta: U64Type) -> U64Type {
//             expressions
//                 .iter()
//                 .cloned()
//                 .fold(U64Type::one(), |acc, expression| acc * theta + expression)
//         }

//         let theta = U64Type(2);
//         let input = vec![U64Type::one(), U64Type(2), U64Type(3)];
//         let result = compress_expressions(&input, theta);
//         println!("Result: {:?}", result);
//     }

//     #[test]
//     fn test3() {}
// }

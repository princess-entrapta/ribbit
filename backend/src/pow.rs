pub trait PowValidator {
    fn is_valid_pow(
        &self,
        challenges: [String; 16],
    ) -> impl std::future::Future<Output = bool> + std::marker::Send;
}

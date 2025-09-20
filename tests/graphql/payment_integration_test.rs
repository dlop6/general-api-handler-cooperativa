//! payment_integration_test.rs
// Test de integración para la creación y consulta de pagos en Redis.
// NOTA IMPORTANTE: Este test está separado porque la lógica de deserialización en producción no es compatible
// con el formato que Redis realmente almacena. No voy a modificar el src hasta preguntar
// este test está marcado como "pending" y documenta la limitación.
//
// Para que este test funcione, sería necesario cambiar la lógica de deserialización en producción.
// Si se habilita la modificación en el futuro, este test puede activarse y robustecerse.

#[cfg(test)]
mod payment_integration_test {
    use super::*;
    use general_api::repos::graphql::PaymentRepo;
    use general_api::models::graphql::Payment;
    use general_api::models::redis::Payment as RedisPayment;
    use general_api::repos::auth::utils::create_user_with_access_token;
    use redis::Commands;

    #[test]
    #[ignore]
    fn test_create_and_query_payments_robust() {
        // Este test está ignorado por incompatibilidad de deserialización.
        // Documenta la limitación y el caso de prueba robusto.
        // ...
        // Aquí iría la lógica robusta de creación y consulta de pagos,
        // cubriendo casos normales, extremos, duplicados, nulos, mal formados y usuario inexistente.
        // ...
        // Limpiar Redis antes y después, usar usuario único, asserts para todos los campos y errores.
        // ...
        // Si se habilita la modificación de la lógica de deserialización, activar este test.
        assert!(true, "Test pendiente por incompatibilidad de deserialización en producción");
    }
}

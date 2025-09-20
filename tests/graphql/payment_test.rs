use general_api::{
    endpoints::handlers::configs::connection_pool::get_pool_connection,
    models::redis::Payment as RedisPayment,
    repos::{
        auth::{create_user_with_access_token, utils::hashing_composite_key},
        graphql::payment::PaymentRepo,
    },
};
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};
use redis::{Commands, JsonCommands};
use dotenv;



/// Usuario sin pagos
#[test]
fn test_get_user_payments_empty_result() {
    // Cargar variables de entorno del .env
    dotenv::dotenv().ok();
    let repo = PaymentRepo {
        pool: get_pool_connection(),
    };

    let random_string = Alphanumeric.sample_string(&mut rng(), 16);
    let access_token = create_user_with_access_token(
        random_string.clone(),
        random_string.clone(),
        format!("Empty User {}", random_string),
    )
    .unwrap()
    .access_token;

    let result = repo.get_user_payments(access_token);
    assert!(result.is_ok(), "Debe ejecutarse correctamente sin pagos");
    assert_eq!(result.unwrap().len(), 0, "Debe retornar lista vacía");
}

/// Token inválido
#[test]
fn test_get_user_payments_invalid_token() {
    // Cargar variables de entorno del .env
    dotenv::dotenv().ok();
    let repo = PaymentRepo {
        pool: get_pool_connection(),
    };

    let invalid_token = "token_inexistente_12345".to_string();
    let result = repo.get_user_payments(invalid_token);

    assert!(result.is_ok(), "Debe manejarse correctamente token inválido");
    assert_eq!(result.unwrap().len(), 0, "Debe retornar lista vacía");
}
// Test unitario para PaymentRepo::get_user_history
// SCRUM-204: Test  de historial de pagos de usuario

use general_api::models::graphql::PaymentHistory;
use general_api::repos::graphql::payment::PaymentRepo;
use general_api::repos::auth::utils::hashing_composite_key;
use r2d2::Pool;
use redis::{Client, Commands};
use actix_web::web;

fn setup_redis_for_test(pool: &web::Data<Pool<Client>>, db_access_token: &str, payed: &str, owed: &str) {
    let mut con = pool.get().expect("Couldn't connect to pool");
    let _ = con.set::<_, _, ()>(format!("users:{}:payed_to_capital", db_access_token), payed);
    let _ = con.set::<_, _, ()>(format!("users:{}:owed_capital", db_access_token), owed);
}

fn cleanup_redis_for_test(pool: &web::Data<Pool<Client>>, db_access_token: &str) {
    let mut con = pool.get().expect("Couldn't connect to pool");
    let _ = con.del::<_, ()>(format!("users:{}:payed_to_capital", db_access_token));
    let _ = con.del::<_, ()>(format!("users:{}:owed_capital", db_access_token));
}

#[test]
fn test_get_user_history_cases() {
    // Setup pool y usuario único
    let client = Client::open("redis://127.0.0.1/").expect("Failed to open Redis");
    let pool = Pool::builder().build(client).expect("Failed to build pool");
    let pool_data = web::Data::new(pool);
    let access_token = format!("test_token_{}", rand::random::<u32>());
    let db_access_token = hashing_composite_key(&[&access_token]);
    let repo = PaymentRepo { pool: pool_data.clone() };

    // Casos de prueba
    let cases = vec![
        ("1000.50", "500.25", Some((1000.50, 500.25))), // normal
        ("0", "0", Some((0.0, 0.0))), // cero
        ("-100", "-50", Some((-100.0, -50.0))), // negativos
        ("1000000000", "500000000", Some((1e9, 5e8))), // grandes
        ("abc", "xyz", Some((0.0, 0.0))), // corruptos
    ];

    for (payed, owed, expected) in cases {
        // Limpieza previa
        cleanup_redis_for_test(&pool_data, &db_access_token);
        // Seteo de datos
        setup_redis_for_test(&pool_data, &db_access_token, payed, owed);
        // Test
        let result = repo.get_user_history(access_token.clone());
        match expected {
            Some((exp_payed, exp_owed)) => {
                let history = result.expect("Debe retornar Ok en caso válido/corrupto");
                assert_eq!(history.payed_to_capital, exp_payed, "payed_to_capital incorrecto");
                assert_eq!(history.owed_capital, exp_owed, "owed_capital incorrecto");
            }
            None => {
                assert!(result.is_err(), "Debe retornar Err en caso de usuario inexistente");
            }
        }
        // Limpieza posterior
        cleanup_redis_for_test(&pool_data, &db_access_token);
    }

    // Caso usuario inexistente
    let result = repo.get_user_history("usuario_inexistente".to_string());
    assert!(result.is_err(), "Debe retornar Err para usuario inexistente");
}

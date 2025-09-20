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

/// SCRUM-201: Test unitario para query de pagos por socio
#[test]
fn test_get_user_payments_by_socio() {
    dotenv::dotenv().ok();

    let repo = PaymentRepo {
        pool: get_pool_connection(),
    };

    // Crear usuario único para el test
    let mut random_string = Alphanumeric.sample_string(&mut rng(), 16);
    let access_token = loop {
        match create_user_with_access_token(
            random_string.clone(),
            random_string.clone(),
            format!("Test User {}", random_string),
        ) {
            Ok(token_info) => break token_info.access_token,
            Err(_) => {
                random_string = Alphanumeric.sample_string(&mut rng(), 16);
            }
        }
    };

    let db_access_token = hashing_composite_key(&[&access_token]);
    let mut con = get_pool_connection().into_inner().get().unwrap();

    // Datos de prueba
    let test_payments = vec![
        RedisPayment {
            date_created: "2024-01-15".to_string(),
            comprobante_bucket: "bucket/payment1.jpg".to_string(),
            ticket_number: "TK001234".to_string(),
            status: "APPROVED".to_string(),
            quantity: 1500.50,
            comments: "Pago de cuota mensual".to_string(),
        },
        RedisPayment {
            date_created: "2024-02-15".to_string(),
            comprobante_bucket: "bucket/payment2.jpg".to_string(),
            ticket_number: "TK005678".to_string(),
            status: "PENDING".to_string(),
            quantity: 750.25,
            comments: "Pago parcial pendiente de revisión".to_string(),
        },
        RedisPayment {
            date_created: "2024-03-15".to_string(),
            comprobante_bucket: "bucket/payment3.jpg".to_string(),
            ticket_number: "TK009876".to_string(),
            status: "REJECTED".to_string(),
            quantity: 2000.00,
            comments: "Pago rechazado por documento ilegible".to_string(),
        },
    ];

    // Insertar pagos en Redis en el formato exacto que espera el código de producción
    // Basado en el diagnóstico: el código espera recibir un array directamente, no un string
    for (index, test_payment) in test_payments.iter().enumerate() {
        let payment_key = format!("users:{}:payments:payment_{}", db_access_token, index + 1);
        
        // Crear el array que contiene el payment, como espera el código de producción
        let payment_array = vec![test_payment.clone()];
        
        // Insertar directamente el array JSON, no como string serializado
        con.json_set::<String, &str, Vec<RedisPayment>, ()>(payment_key, "$", &payment_array)
            .expect("Failed to set payment in Redis");
    }

    // Acción: ejecutar query
    let result = repo.get_user_payments(access_token.clone());
    assert!(result.is_ok(), "La query debe ejecutarse exitosamente");
    let payments = result.unwrap();

    assert_eq!(
        payments.len(),
        test_payments.len(),
        "Debe retornar el número correcto de pagos"
    );

    for (index, payment) in payments.iter().enumerate() {
        let expected_payment = &test_payments[index];

        assert!(
            !payment.payment_id.is_empty(),
            "payment_id no debe estar vacío en pago {}",
            index + 1
        );
        assert!(
            payment.payment_id.starts_with("payment_"),
            "payment_id debe tener formato correcto en pago {}",
            index + 1
        );
        assert_eq!(
            payment.total_amount, expected_payment.quantity,
            "total_amount debe coincidir en pago {}",
            index + 1
        );
        assert_eq!(
            payment.payment_date, expected_payment.date_created,
            "payment_date debe coincidir en pago {}",
            index + 1
        );
        assert_eq!(
            payment.ticket_num, expected_payment.ticket_number,
            "ticket_num debe coincidir en pago {}",
            index + 1
        );
        assert_eq!(
            payment.commentary, expected_payment.comments,
            "commentary debe coincidir en pago {}",
            index + 1
        );
        assert_eq!(
            payment.photo, expected_payment.comprobante_bucket,
            "photo debe coincidir en pago {}",
            index + 1
        );
        assert_eq!(
            payment.state, expected_payment.status,
            "state debe coincidir en pago {}",
            index + 1
        );
    }

    // Cleanup
    for index in 0..test_payments.len() {
        let payment_key = format!("users:{}:payments:payment_{}", db_access_token, index + 1);
        let _: Result<(), redis::RedisError> = con.del(payment_key);
    }

    println!("SCRUM-201 completado con {} pagos validados", payments.len());
}

/// Usuario sin pagos
#[test]
fn test_get_user_payments_empty_result() {
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
    let repo = PaymentRepo {
        pool: get_pool_connection(),
    };

    let invalid_token = "token_inexistente_12345".to_string();
    let result = repo.get_user_payments(invalid_token);

    assert!(result.is_ok(), "Debe manejarse correctamente token inválido");
    assert_eq!(result.unwrap().len(), 0, "Debe retornar lista vacía");
}
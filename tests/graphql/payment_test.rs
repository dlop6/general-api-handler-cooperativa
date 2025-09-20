use general_api::{use general_api::{

    endpoints::handlers::configs::connection_pool::get_pool_connection,    endpoints::handlers::configs::connection_pool::get_pool_connection,

    models::redis::Payment as RedisPayment,    models::redis::Payment as RedisPayment,

    repos::{    repos::{

        auth::{create_user_with_access_token, utils::hashing_composite_key},        auth::{create_user_with_access_token, utils::hashing_composite_key},

        graphql::payment::PaymentRepo,        graphql::payment::PaymentRepo,

    },    },

};};

use rand::{use rand::{

    distr::{Alphanumeric, SampleString},    distr::{Alphanumeric, SampleString},

    rng,    rng,

};};

use redis::{Commands, JsonCommands};use redis::{Commands, JsonCommands};

use dotenv;

use dotenv;

#[test]

fn test_get_user_payments_by_socio() {/// SCRUM-201: Test unitario para query de pagos por socio

    dotenv::dotenv().ok();#[test]

    let repo = PaymentRepo {fn test_get_user_payments_by_socio() {

        pool: get_pool_connection(),    dotenv::dotenv().ok();

    };

    let mut random_string = Alphanumeric.sample_string(&mut rng(), 16);    let repo = PaymentRepo {

    let access_token = loop {        pool: get_pool_connection(),

        match create_user_with_access_token(    };

            random_string.clone(),

            random_string.clone(),    // Crear usuario único para el test

            format!("Test User {}", random_string),    let mut random_string = Alphanumeric.sample_string(&mut rng(), 16);

        ) {    let access_token = loop {

            Ok(token_info) => break token_info.access_token,        match create_user_with_access_token(

            Err(_) => {            random_string.clone(),

                random_string = Alphanumeric.sample_string(&mut rng(), 16);            random_string.clone(),

            }            format!("Test User {}", random_string),

        }        ) {

    };            Ok(token_info) => break token_info.access_token,

    let db_access_token = hashing_composite_key(&[&access_token]);            Err(_) => {

    let mut con = get_pool_connection().into_inner().get().unwrap();                random_string = Alphanumeric.sample_string(&mut rng(), 16);

    let test_payments = vec![            }

        RedisPayment {        }

            date_created: "2024-01-15".to_string(),    };

            comprobante_bucket: "bucket/payment1.jpg".to_string(),

            ticket_number: "TK001234".to_string(),    let db_access_token = hashing_composite_key(&[&access_token]);

            status: "APPROVED".to_string(),    let mut con = get_pool_connection().into_inner().get().unwrap();

            quantity: 1500.50,

            comments: "Pago de cuota mensual".to_string(),    // Datos de prueba

        },    let test_payments = vec![

        RedisPayment {        RedisPayment {

            date_created: "2024-02-15".to_string(),            date_created: "2024-01-15".to_string(),

            comprobante_bucket: "bucket/payment2.jpg".to_string(),            comprobante_bucket: "bucket/payment1.jpg".to_string(),

            ticket_number: "TK005678".to_string(),            ticket_number: "TK001234".to_string(),

            status: "PENDING".to_string(),            status: "APPROVED".to_string(),

            quantity: 750.25,            quantity: 1500.50,

            comments: "Pago parcial pendiente de revisión".to_string(),            comments: "Pago de cuota mensual".to_string(),

        },        },

        RedisPayment {        RedisPayment {

            date_created: "2024-03-15".to_string(),            date_created: "2024-02-15".to_string(),

            comprobante_bucket: "bucket/payment3.jpg".to_string(),            comprobante_bucket: "bucket/payment2.jpg".to_string(),

            ticket_number: "TK009876".to_string(),            ticket_number: "TK005678".to_string(),

            status: "REJECTED".to_string(),            status: "PENDING".to_string(),

            quantity: 2000.00,            quantity: 750.25,

            comments: "Pago rechazado por documento ilegible".to_string(),            comments: "Pago parcial pendiente de revisión".to_string(),

        },        },

    ];        RedisPayment {

    // Guardar cada pago como string JSON en Redis            date_created: "2024-03-15".to_string(),

    for (index, test_payment) in test_payments.iter().enumerate() {            comprobante_bucket: "bucket/payment3.jpg".to_string(),

        let payment_key = format!("users:{}:payments:payment_{}", db_access_token, index + 1);            ticket_number: "TK009876".to_string(),

        let payment_json_string = serde_json::to_string(&test_payment).expect("Failed to serialize payment");            status: "REJECTED".to_string(),

        con.json_set::<String, &str, String, ()>(payment_key, "$", &payment_json_string)            quantity: 2000.00,

            .expect("Failed to set payment in Redis");            comments: "Pago rechazado por documento ilegible".to_string(),

    }        },

    let result = repo.get_user_payments(access_token.clone());    ];

    assert!(result.is_ok(), "La query debe ejecutarse exitosamente");

    let payments = result.unwrap();    // Insertar pagos en Redis en el formato correcto: "[{...}]"

    assert_eq!(payments.len(), test_payments.len(), "Debe retornar el número correcto de pagos");    for (index, test_payment) in test_payments.iter().enumerate() {

    for (index, payment) in payments.iter().enumerate() {        let payment_key = format!("users:{}:payments:payment_{}", db_access_token, index + 1);

        let expected_payment = &test_payments[index];        let payment_array = vec![test_payment.clone()];

        assert!(!payment.payment_id.is_empty(), "payment_id no debe estar vacío en pago {}", index + 1);        let payment_json_string = serde_json::to_string(&payment_array).expect("Failed to serialize payment array");

        assert_eq!(payment.monto_total, expected_payment.quantity, "monto_total debe coincidir en pago {}", index + 1);        let redis_value = format!("[\"{}\"]", payment_json_string);

        assert_eq!(payment.fecha_pago, expected_payment.date_created, "fecha_pago debe coincidir en pago {}", index + 1);

        assert_eq!(payment.num_boleta, expected_payment.ticket_number, "num_boleta debe coincidir en pago {}", index + 1);        con.json_set::<String, &str, String, ()>(payment_key, "$", &redis_value)

        assert_eq!(payment.comentarios, expected_payment.comments, "comentarios debe coincidir en pago {}", index + 1);            .expect("Failed to set payment in Redis");

        assert_eq!(payment.foto, expected_payment.comprobante_bucket, "foto debe coincidir en pago {}", index + 1);    }

        assert_eq!(payment.estado, expected_payment.status, "estado debe coincidir en pago {}", index + 1);

    }    // Acción: ejecutar query

    // Cleanup    let result = repo.get_user_payments(access_token.clone());

    for index in 0..test_payments.len() {    assert!(result.is_ok(), "La query debe ejecutarse exitosamente");

        let payment_key = format!("users:{}:payments:payment_{}", db_access_token, index + 1);    let payments = result.unwrap();

        let _: Result<(), redis::RedisError> = con.del(payment_key);

    }    assert_eq!(

    println!("SCRUM-201 completado con {} pagos validados", payments.len());        payments.len(),

}        test_payments.len(),

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

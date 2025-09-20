#[cfg(test)]
mod tests {
    use general_api::models::graphql::Affiliate;
    use general_api::repos::graphql::payment::PaymentRepo;
    use std::env;
    use dotenv::dotenv;
    use redis::{Client, Commands};
    use r2d2::Pool;
    use actix_web::web;

    fn setup_redis_test_data(con: &mut redis::Connection) {
        let _ = con.set::<_, _, ()>("affiliate_ids:101", "Juan Perez");
        let _ = con.set::<_, _, ()>("affiliate_ids:202", "Maria Gomez");
    }

    fn cleanup_redis_test_data(con: &mut redis::Connection) {
        let _ = con.del::<_, ()>("affiliate_ids:101");
        let _ = con.del::<_, ()>("affiliate_ids:202");
    }

    #[test]
    fn test_get_all_users_for_affiliates() {
    // Setup pool and connection
    dotenv().ok();
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL not set in .env");
    let client = Client::open(redis_url).expect("Failed to create Redis client");
    let pool = Pool::builder().build(client).expect("Failed to create Redis pool");
    let mut con = pool.get().expect("Couldn't connect to pool");

        // Insert test data
        setup_redis_test_data(&mut con);

        // Instanciar repo y ejecutar función
        let repo = PaymentRepo::init(web::Data::new(pool));
        let result = repo.get_all_users_for_affiliates();

        // Validar resultado
        assert!(result.is_ok(), "La función debe retornar Ok");
        let affiliates = result.unwrap();
        assert_eq!(affiliates.len(), 2, "Debe retornar dos afiliados");

        // Validar datos específicos
        let mut found_juan = false;
        let mut found_maria = false;
        for aff in affiliates {
            match (aff.usuario_id, aff.name.as_str()) {
                (101, "Juan Perez") => found_juan = true,
                (202, "Maria Gomez") => found_maria = true,
                _ => (),
            }
        }
        assert!(found_juan, "Debe encontrar a Juan Perez con id 101");
        assert!(found_maria, "Debe encontrar a Maria Gomez con id 202");

        // Limpiar datos de prueba
        cleanup_redis_test_data(&mut con);
    }
}

#[cfg(test)]
mod nodes {

    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver, Node},
    };
    use flowrs_std::http::HttpNode;
    use serde_json::json;


    #[test]
    fn get_request() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = "GET";
        let path = "/hello&question=how%20are%20you";
        let expected_response_body = "Hello World!";

        // Create a mock
        let mock = server
            .mock(&method, path.clone())
            .with_status(200)
            .with_body(expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();
        let data_input_json = json!({
        "url": url.to_string() + path,
        "method": method
        });
     
        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input_json.clone());
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let returned_body = mock_output.next().unwrap();
        assert!(returned_body == expected_response_body, "expected_body: {}, returned_body: {}", expected_response_body, returned_body);
    }

    #[test]
    fn get_request_with_headers_and_body() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = "GET";
        let path = "/hello";
        let content_type_header = ("content-type", "text/plain");
        let x_api_key_header = ("x-api-key", "1234");
        let expected_response_body = "AI";

        // Create a mock
        let mock = server
            .mock(&method, path.clone())
            .with_status(200)
            .match_header(&content_type_header.0, content_type_header.1)
            .match_header(&x_api_key_header.0, x_api_key_header.1)
            .with_body(&expected_response_body)
            .create();


        let change_observer: ChangeObserver = ChangeObserver::new();
        let data_input_json = json!({
        "url": url.to_string() + path,
        "method": method,
        "headers": {
            content_type_header.0: content_type_header.1,
            x_api_key_header.0: x_api_key_header.1
        },
        "body": expected_response_body
        });
     
        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input_json.clone());
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let returned_body = mock_output.next().unwrap();
        assert!(returned_body == expected_response_body, "expected_body: {}, returned_body: {}", expected_response_body, returned_body);

    }

    #[test]
    fn post_request() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = "post";
        let path = "/post_request";
        let request_body = json!({
            "name": "Post Doe",
            "age": 42
        });
        let expected_response_body = "Hello Post World!";

        // Create a mock server, which also checks the request body.
        let mock = server
            .mock(&method, path.clone())
            .match_body(serde_json::to_string(&request_body).unwrap().as_str())
            .with_status(200)
            .with_body(expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();
        let data_input_json = json!({
        "url": url.to_string() + path,
        "method": method,
        "body": request_body
        });
     
        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input_json.clone());
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let returned_body = mock_output.next().unwrap();
        assert!(returned_body == expected_response_body, "expected_body: {}, returned_body: {}", expected_response_body, returned_body);
    }

    #[test]
    fn get_request_with_an_empty_header_value() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = "GET";
        let path = "/hello";
        let content_type_header = ("content-type", "");
        let x_api_key_header = ("x-api-key", "1234");
        let expected_response_body = "AI";

        // Create a mock
        let mock = server
            .mock(&method, path.clone())
            .with_status(200)
            .match_header(&content_type_header.0, content_type_header.1)
            .match_header(&x_api_key_header.0, x_api_key_header.1)
            .with_body(&expected_response_body)
            .create();


        let change_observer: ChangeObserver = ChangeObserver::new();
        let data_input_json = json!({
        "url": url.to_string() + path,
        "method": method,
        "headers": {
            content_type_header.0: content_type_header.1,
            x_api_key_header.0: x_api_key_header.1
        },
        "body": expected_response_body
        });
     
        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input_json.clone());
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let returned_body = mock_output.next().unwrap();
        assert!(returned_body == expected_response_body, "expected_body: {}, returned_body: {}", expected_response_body, returned_body);

    }
}
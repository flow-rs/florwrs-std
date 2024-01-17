#[cfg(test)]
mod nodes {

    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver, Node},
    };
    use flowrs_std::http::{ConfigInput, HTTPMethod, HttpNode, RequestInput};
    use flowrs_std::nodes::javascript::JsNode;
    use flowrs_std::value::ValueNode;
    use serde_json::{json, Value};
    use std::{collections::HashMap, time::Duration};

    #[test]
    fn get_request() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = HTTPMethod::GET;
        let path = "/hello&question=how%20are%20you";
        let expected_response_body = "Hello World!";

        // Create a mock
        let mock = server
            .mock("GET", path)
            .with_status(200)
            .with_body(expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();
        let data_input = RequestInput {
            url: url.to_string() + path,
            method,
            headers: HashMap::new(),
            body: None,
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input);
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let response = mock_output.next().unwrap();
        assert!(
            response.body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            response.body
        );
    }

    #[test]
    fn get_request_with_timeout_change() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = HTTPMethod::GET;
        let path = "/hello";
        let expected_response_body = "Hello World!";

        // Create a mock
        let mock = server
            .mock("GET", path)
            .with_status(200)
            .with_body(expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();
        let data_input = RequestInput {
            url: url.to_string() + path,
            method,
            headers: HashMap::new(),
            body: None,
        };
        let new_timeout = 5000;
        let config_input = ConfigInput {
            timeout: Some(Duration::from_millis(new_timeout)),
            accept_invalid_certs: Some(false),
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.config_input.send(config_input);
        let _ = http_node.data_input.send(data_input.clone());
        let initial_timeout = http_node.timeout();
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let response = mock_output.next().unwrap();
        assert!(initial_timeout != http_node.timeout());
        assert!(http_node.timeout() == Duration::from_millis(new_timeout));
        assert!(
            response.body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            response.body
        );
    }

    #[test]
    fn get_request_with_headers_and_body() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = HTTPMethod::GET;
        let path = "/hello";
        let content_type_header = ("content-type", "text/plain");
        let x_api_key_header = ("x-api-key", "1234");
        let expected_response_body = "AI";

        // Create a mock
        let mock = server
            .mock("GET", path)
            .with_status(200)
            .match_header(&content_type_header.0, content_type_header.1)
            .match_header(&x_api_key_header.0, x_api_key_header.1)
            .with_body(&expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();

        let data_input = RequestInput {
            url: url.to_string() + path,
            method,
            headers: {
                let mut headers = HashMap::new();
                headers.insert(
                    content_type_header.0.to_string(),
                    content_type_header.1.to_string(),
                );
                headers.insert(
                    x_api_key_header.0.to_string(),
                    x_api_key_header.1.to_string(),
                );
                headers
            },
            body: Some(expected_response_body.to_string()),
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input.clone());
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let response = mock_output.next().unwrap();
        assert!(
            response.body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            response.body
        );
    }

    #[test]
    fn post_request() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = HTTPMethod::POST;
        let path = "/post_request";
        let request_body = json!({
            "name": "Post Doe",
            "age": 42
        });
        let expected_response_body = "Hello Post World!";

        // Create a mock server, which also checks the request body.
        let mock = server
            .mock("POST", path)
            .match_body(serde_json::to_string(&request_body).unwrap().as_str())
            .with_status(200)
            .with_body(expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();

        let data_input = RequestInput {
            url: url.to_string() + path,
            method: method,
            headers: HashMap::new(),
            body: Some(serde_json::to_string(&request_body).unwrap()),
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input.clone());
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let response = mock_output.next().unwrap();
        assert!(
            response.body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            response.body
        );
    }

    #[test]
    fn get_request_with_an_empty_header_value() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = HTTPMethod::GET;
        let path = "/hello";
        let content_type_header = ("content-type", "");
        let x_api_key_header = ("x-api-key", "1234");
        let expected_response_body = "AI";

        // Create a mock
        let mock = server
            .mock("GET", path)
            .with_status(200)
            .match_header(&content_type_header.0, content_type_header.1)
            .match_header(&x_api_key_header.0, x_api_key_header.1)
            .with_body(&expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();

        let data_input = RequestInput {
            url: url.to_string() + path,
            method,
            headers: {
                let mut headers = HashMap::new();
                headers.insert(
                    content_type_header.0.to_string(),
                    content_type_header.1.to_string(),
                );
                headers.insert(
                    x_api_key_header.0.to_string(),
                    x_api_key_header.1.to_string(),
                );
                headers
            },
            body: Some(expected_response_body.to_string()),
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input.clone());
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let response = mock_output.next().unwrap();
        assert!(
            response.body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            response.body
        );
    }

    #[test]
    fn get_request_with_404_response() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = HTTPMethod::GET;
        let path = "/hello";
        let content_type_header = ("content-type", "");
        let x_api_key_header = ("x-api-key", "1234");
        let expected_response_body = "AI";
        let expected_response_code = 404;

        // Create a mock
        let mock = server
            .mock("GET", path)
            .with_status(expected_response_code)
            .match_header(&content_type_header.0, content_type_header.1)
            .match_header(&x_api_key_header.0, x_api_key_header.1)
            .with_body(&expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();

        let data_input = RequestInput {
            url: url.to_string() + path,
            method,
            headers: {
                let mut headers = HashMap::new();
                headers.insert(
                    content_type_header.0.to_string(),
                    content_type_header.1.to_string(),
                );
                headers.insert(
                    x_api_key_header.0.to_string(),
                    x_api_key_header.1.to_string(),
                );
                headers
            },
            body: Some(expected_response_body.to_string()),
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input.clone());
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let response = mock_output.next().unwrap();
        assert!(
            response.body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            response.body
        );
        assert!(response.response_code == expected_response_code as u16);
    }

    #[test]
    fn put_request() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = HTTPMethod::PUT;
        let path = "/hello&question=how%20are%20you";
        let expected_response_body = "Hello World!";

        // Create a mock
        let mock = server
            .mock("PUT", path)
            .with_status(200)
            .with_body(expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();
        let data_input = RequestInput {
            url: url.to_string() + path,
            method,
            headers: HashMap::new(),
            body: None,
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input);
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let response = mock_output.next().unwrap();
        assert!(
            response.body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            response.body
        );
    }

    #[test]
    fn delete_request() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = HTTPMethod::DELETE;
        let path = "/hello&question=how%20are%20you";
        let expected_response_body = "Hello World!";

        // Create a mock
        let mock = server
            .mock("DELETE", path)
            .with_status(200)
            .with_body(expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();
        let data_input = RequestInput {
            url: url.to_string() + path,
            method,
            headers: HashMap::new(),
            body: None,
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input);
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let response = mock_output.next().unwrap();
        assert!(
            response.body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            response.body
        );
    }

    #[test]
    fn head_request() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = HTTPMethod::HEAD;
        let path = "/hello&question=how%20are%20you";
        let expected_response_body = "Hello World!";

        // Create a mock
        let mock = server
            .mock("HEAD", path)
            .with_status(200)
            .with_body(expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();
        let data_input = RequestInput {
            url: url.to_string() + path,
            method,
            headers: HashMap::new(),
            body: None,
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input);
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let return_object = mock_output.next().unwrap();
        assert!(
            return_object.body == "",
            "Returned body should be empty, as it is a head request, actual returned_body: {}",
            return_object.body
        );
        assert!(
            return_object.response_code == 200,
            "expected response code: {}, actual response code: {}",
            200,
            return_object.response_code
        );
    }

    #[test]
    fn patch_request() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = HTTPMethod::PATCH;
        let path = "/hello&question=how%20are%20you";
        let expected_response_body = "Hello World!";

        // Create a mock
        let mock = server
            .mock("PATCH", path)
            .with_status(200)
            .with_body(expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();
        let data_input = RequestInput {
            url: url.to_string() + path,
            method,
            headers: HashMap::new(),
            body: None,
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input);
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let response = mock_output.next().unwrap();
        assert!(
            response.body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            response.body
        );
    }

    #[test]
    fn options_request() {
        let mut server = mockito::Server::new();

        let url = server.url();
        let method = HTTPMethod::OPTIONS;
        let path = "/hello&question=how%20are%20you";
        let expected_response_body = "Hello World!";

        // Create a mock
        let mock = server
            .mock("OPTIONS", path)
            .with_status(200)
            .with_body(expected_response_body)
            .create();

        let change_observer: ChangeObserver = ChangeObserver::new();
        let data_input = RequestInput {
            url: url.to_string() + path,
            method,
            headers: HashMap::new(),
            body: None,
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.data_input.send(data_input);
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let response = mock_output.next().unwrap();
        assert!(
            response.body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            response.body
        );
    }

    #[test]
    #[ignore]
    fn post_request_starcoder() {
        let url = "http://10.28.229.17:3005/v1/generate";
        let method = HTTPMethod::POST;
        let request_body = json!({
          "prompt": "Provide a Hello World code in Python"
        });

        let change_observer: ChangeObserver = ChangeObserver::new();

        let data_input = RequestInput {
            url: url.to_string(),
            method: method,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());
                headers
            },
            body: Some(request_body.to_string()),
        };

        let config_input = ConfigInput {
            accept_invalid_certs: Some(true),
            timeout: Some(Duration::from_secs(15)),
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.config_input.send(config_input);
        let _ = http_node.data_input.send(data_input);
        http_node.on_update().unwrap();

        let extended_output = mock_output.next().unwrap();
        assert!(200 == extended_output.response_code);
    }

    /// Test for presenation purposes to show the complete flow of the Coding Assistant
    #[test]
    #[ignore]
    fn complete_flow_test() {

        // JavaScript which constructs the request object for HTTP Node
        let code = r#"function main(input) { 
            return { 
                url: "http://10.28.229.17:3006/v1/generate", 
                method: "POST", 
                headers: {"content-type": "application/json"}, 
                body: "{ \"prompt\": \"" + input + "\",  \"llm_config\": { \"temperature\": 0.2, \"max_new_tokens\": 500 }}"} ;
        }"#;

        // Prompt for Coding Assistant backend
        let value_input = "def list_comprehension_example():";

        let change_observer: ChangeObserver = ChangeObserver::new();

        // ConfigInput object which configures the HTTP Node
        let config_input = ConfigInput {
            accept_invalid_certs: Some(true),
            timeout: Some(Duration::from_secs(15)),
        };

        // Construction of all necessary nodes for test
        let code_value_node: ValueNode<String> =
            ValueNode::new(code.to_string(), Some(&change_observer));
        let prompt_value_node: ValueNode<String> =
            ValueNode::new(value_input.to_string(), Some(&change_observer));
        let mut js_node: JsNode<String, RequestInput> = JsNode::new(Some(&change_observer));
        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));

        // Connection of all nodes
        let mock_output = Edge::new();
        connect(code_value_node.output.clone(), js_node.code_input.clone());
        connect(prompt_value_node.output.clone(), js_node.input.clone());
        connect(js_node.output.clone(), http_node.data_input.clone());
        connect(http_node.output.clone(), mock_output.clone());

        // Executing the complete flow to send a request to the Coding Assistant backend
        http_node.config_input.send(config_input).unwrap();
        code_value_node.on_ready().unwrap();
        prompt_value_node.on_ready().unwrap();
        js_node.on_update().unwrap();
        http_node.on_update().unwrap();

        let extended_output = mock_output.next().unwrap();
        // print!("{}", extended_output.body);
        let res_str = extended_output.body.as_str();
        let response_json: Value = serde_json::from_str(res_str).unwrap();
        let response = response_json["outputs"][0]["text"].to_string().replace("\\n", "\n");
        print!("Response:\n\n{}\n\n", response);
        
        assert!(200 == extended_output.response_code);
    }
}

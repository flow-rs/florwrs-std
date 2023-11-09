#[cfg(test)]
mod nodes {

    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver, Node},
    };
    use flowrs_std::http::{ConfigInput, HTTPMethod, HttpNode, RequestInput};
    use serde_json::json;
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
            .mock("GET", path.clone())
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
        connect(http_node.body_output.clone(), mock_output.clone());
        let extended_output = Edge::new();
        connect(http_node.extended_output.clone(), extended_output.clone());
        let _ = http_node.data_input.send(data_input);
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let returned_body = mock_output.next().unwrap();
        assert!(
            returned_body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            returned_body
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
            .mock("GET", path.clone())
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
        connect(http_node.body_output.clone(), mock_output.clone());
        let extended_output = Edge::new();
        connect(http_node.extended_output.clone(), extended_output.clone());
        let _ = http_node.config_input.send(config_input);
        let _ = http_node.data_input.send(data_input.clone());
        let initial_timeout = http_node.timeout;
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let returned_body = mock_output.next().unwrap();
        assert!(initial_timeout != http_node.timeout);
        assert!(http_node.timeout == Duration::from_millis(new_timeout));
        assert!(
            returned_body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            returned_body
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
            .mock("GET", path.clone())
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
        connect(http_node.body_output.clone(), mock_output.clone());
        let extended_output = Edge::new();
        connect(http_node.extended_output.clone(), extended_output.clone());
        let _ = http_node.data_input.send(data_input.clone());
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let returned_body = mock_output.next().unwrap();
        assert!(
            returned_body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            returned_body
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
            .mock("POST", path.clone())
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
        connect(http_node.body_output.clone(), mock_output.clone());
        let extended_output = Edge::new();
        connect(http_node.extended_output.clone(), extended_output.clone());
        let _ = http_node.data_input.send(data_input.clone());
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let returned_body = mock_output.next().unwrap();
        assert!(
            returned_body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            returned_body
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
            .mock("GET", path.clone())
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
        connect(http_node.body_output.clone(), mock_output.clone());
        let extended_output = Edge::new();
        connect(http_node.extended_output.clone(), extended_output.clone());
        let _ = http_node.data_input.send(data_input.clone());
        http_node.on_update().unwrap();

        mock.assert(); // checks if the mock server has been called
        let returned_body = mock_output.next().unwrap();
        assert!(
            returned_body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            returned_body
        );
    }

    #[test]
    #[ignore]
    fn post_request_starcoder() {
        let url = "https://leonambaum-fastapi.lab.kube.cs.hm.edu/v1/generate";
        let method = HTTPMethod::POST;
        let request_body = json!({
          "prompt": "Provide a Hello World code in Python",
          "llm_config": {
            "max_new_tokens": 256,
            "min_length": 0,
            "min_new_tokens": 32,
            "early_stopping": false,
            "num_beams": 1,
            "num_beam_groups": 1,
            "use_cache": true,
            "temperature": 0.2,
            "top_k": 50,
            "top_p": 0.95,
            "typical_p": 1,
            "epsilon_cutoff": 0,
            "eta_cutoff": 0,
            "diversity_penalty": 0,
            "repetition_penalty": 1.2,
            "encoder_repetition_penalty": 1,
            "length_penalty": 1,
            "no_repeat_ngram_size": 0,
            "renormalize_logits": false,
            "remove_invalid_values": false,
            "num_return_sequences": 1,
            "output_attentions": false,
            "output_hidden_states": false,
            "output_scores": false,
            "pad_token_id": 49152,
            "encoder_no_repeat_ngram_size": 0,
            "n": 1,
            "presence_penalty": 0,
            "frequency_penalty": 0,
            "use_beam_search": false,
            "ignore_eos": false
          },
          "adapter_name": null
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
            timeout: Some(Duration::from_secs(40)),
        };

        let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        let extended_mock_output = Edge::new();
        connect(http_node.body_output.clone(), mock_output.clone());
        connect(
            http_node.extended_output.clone(),
            extended_mock_output.clone(),
        );
        let _ = http_node.config_input.send(config_input);
        let _ = http_node.data_input.send(data_input);
        http_node.on_update().unwrap();

        let _ = mock_output.next().unwrap();
        let extended_output = extended_mock_output.next().unwrap();
        assert!(200 == extended_output.response_code);
    }
}

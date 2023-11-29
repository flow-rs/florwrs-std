#[cfg(test)]
mod nodes {

    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver, Node},
    };
    use flowrs_std::http::{ConfigInput, HTTPMethod, HttpNode, RequestInput};
    use serde_json::json;
    use serde_json::Value;
    use std::{collections::HashMap, time::Duration};

    #[test]
    fn normal_get_request() {
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
        let response_output = mock_output.next().unwrap();
        assert!(
            response_output.body == expected_response_body,
            "expected_body: {}, returned_body: {}",
            expected_response_body,
            response_output.body
        );
    }

    #[test]
    fn post_request_starcoder_llm() {
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
        connect(http_node.output.clone(), mock_output.clone());
        let _ = http_node.config_input.send(config_input);
        let _ = http_node.data_input.send(data_input);
        http_node.on_update().unwrap();

        let response_output = mock_output.next().unwrap();
        let response_body: Value = serde_json::from_str(&response_output.body).unwrap();
        println!(
            "\n\nResponse from LLM: {}\n\n",
            response_body["responses"][0]["text"]
        );
        assert!(200 == response_output.response_code);
    }
}

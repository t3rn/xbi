t0rn-rococo:
	cargo run -- -d \
		--primary_node_host wss://dev.net.t3rn.io \
		--primary_node_key /tmp/key \
		-s '[{"id":1000,"host":"wss://rococo-rpc.polkadot.io","sleep_time_secs":5,"listener_events":[{"module": "Ump","variant": "ExecutedUpward"}]},{"id":1000,"host":"wss://dev.net.t3rn.io","sleep_time_secs":5,"listener_events":[{"module": "System","variant": "ExtrinsicFailed"}]}]'
t0rn-local:
	cargo run -- -d \
		--primary_parachain_id 2000 \
		--primary_node_host ws://localhost:9847 \
		--secondary_parachain_id 3000 \
		--secondary_node_host ws://localhost:9647 \
		-s '[{"id":1000,"host":"ws://localhost:9944","sleep_time_secs":5,"listener_events":[{"module": "Ump","variant": "ExecutedUpward"}]},{"id":1000,"host":"ws://localhost:9847","sleep_time_secs":5,"listener_events":[{"module": "System","variant": "ExtrinsicFailed"}]},{"id":1000,"host":"ws://localhost:9647","sleep_time_secs":5,"listener_events":[{"module": "System","variant": "ExtrinsicFailed"}]}]'
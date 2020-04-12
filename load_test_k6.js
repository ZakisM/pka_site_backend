import http from 'k6/http';
import { check, sleep } from 'k6';

export default function() {
	var url = 'http://localhost:1234/v1/api/search_pka_event';

	var payload = JSON.stringify({
		query: 'woody',
	});

	var params = {
		headers: {
			'content-type': 'application/json',
		},
	};

	let res = http.post(url, payload, params);

	check(res, { 'status was 200': r => r.status = 200 });

	sleep(1);
}

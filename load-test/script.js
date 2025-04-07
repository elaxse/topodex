import http from "k6/http";
import { check } from "k6";
import { Counter } from "k6/metrics";

const locations_resolved = new Counter("locations_resolved");

export const options = {
  scenarios: {
    warmup: {
      executor: "constant-vus",
      vus: 10,
      duration: "20s",
    },
    load_test: {
      executor: "constant-vus",
      vus: 10,
      duration: "20s",
      startTime: "20s",
    },
  },
  thresholds: {
    "iterations{scenario:load_test}": [],
    "iterations{scenario:warmup}": [],
  },
};

export default function () {
  const locationsPerRequest = 200;
  const request = () => ({
    lat: Math.random() * 180 - 90,
    lng: Math.random() * 360 - 180,
  });
  const req = {
    locations: Array(locationsPerRequest)
      .fill()
      .map(() => request()),
  };
  let res = http.post("http://0.0.0.0:8090/lookup", JSON.stringify(req), {
    headers: { "Content-Type": "application/json" },
  });
  check(res, { "status is 200": (res) => res.status === 200 });
  locations_resolved.add(locationsPerRequest);
}

import http from "k6/http";
import { sleep, check } from "k6";

export const options = {
  vus: 10,
  duration: "30s",
};

export default function () {
  const request = () => ({
    lat: Math.random() * 180 - 90,
    lng: Math.random() * 360 - 180,
  });
  const req = {
    locations: Array(200)
      .fill()
      .map(() => request()),
  };
  let res = http.post("http://0.0.0.0:8090/lookup", JSON.stringify(req), {
    headers: { "Content-Type": "application/json" },
  });
  check(res, { "status is 200": (res) => res.status === 200 });
}

import http from "k6/http";
import { sleep, check } from "k6";

export const options = {
  vus: 100,
  duration: "10s",
};

export default function () {
  const lat = Math.random() * 180 - 90;
  const lng = Math.random() * 360 - 180;
  const andorra_request = {
    lat: 42.5524789,
    lng: 1.5716917,
  };
  const req = {
    locations: Array(750)
      .fill()
      .map(() => andorra_request),
  };
  for (let i = 0; i < 100; i++) {
    let res = http.post("http://127.0.0.1:8090/lookup", JSON.stringify(req), {
      headers: { "Content-Type": "application/json" },
    });
    check(res, { "status is 200": (res) => res.status === 200 });
  }
}

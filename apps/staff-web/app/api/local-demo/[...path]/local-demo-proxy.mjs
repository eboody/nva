const allowedPathRoot = "v1";

export function safeLocalDemoApiPath(segments) {
  if (segments[0] !== allowedPathRoot) return undefined;
  for (const segment of segments) {
    if (!segment || segment === "." || segment === ".." || segment.includes("/")) {
      return undefined;
    }
  }
  return segments.map((segment) => encodeURIComponent(segment)).join("/");
}

export async function fetchLocalDemoApi({
  apiBaseUrl,
  segments,
  search,
  method,
  headers,
  fetchImpl = fetch
}) {
  const safePath = safeLocalDemoApiPath(segments);
  if (!safePath) {
    return Response.json(
      {
        error: {
          code: "unsupported_local_demo_api_path",
          message: "Only /v1 local demo API paths are proxied."
        },
        live_side_effects_allowed: false
      },
      { status: 404 }
    );
  }

  const upstreamUrl = new URL(`${apiBaseUrl}/`);
  upstreamUrl.pathname = `${upstreamUrl.pathname.replace(/\/$/, "")}/${safePath}`;
  upstreamUrl.search = search;
  return fetchImpl(upstreamUrl, {
    method,
    headers,
    cache: "no-store"
  });
}

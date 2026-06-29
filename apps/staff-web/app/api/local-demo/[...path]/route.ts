import { NextRequest, NextResponse } from "next/server";

const localDemoApiBaseUrlEnvName = "PET_RESORT_API_BASE_URL";
const allowedPathRoot = "v0";

function configuredApiBaseUrl(): string | undefined {
  const value = process.env[localDemoApiBaseUrlEnvName]?.trim();
  return value && /^https?:\/\//.test(value) ? value.replace(/\/$/, "") : undefined;
}

function unavailable(message: string) {
  return NextResponse.json(
    {
      error: {
        code: "local_demo_api_unavailable",
        message
      },
      live_side_effects_allowed: false
    },
    { status: 503 }
  );
}

function safeLocalDemoApiPath(segments: string[]): string | undefined {
  if (segments[0] !== allowedPathRoot) {
    return undefined;
  }

  for (const segment of segments) {
    if (!segment || segment === "." || segment === ".." || segment.includes("/")) {
      return undefined;
    }
  }

  return segments.map((segment) => encodeURIComponent(segment)).join("/");
}

export async function GET(
  request: NextRequest,
  context: { params: Promise<{ path?: string[] }> }
) {
  const params = await context.params;
  const path = safeLocalDemoApiPath(params.path ?? []);

  if (!path) {
    return NextResponse.json(
      {
        error: {
          code: "unsupported_local_demo_api_path",
          message: "Only /v0 local demo API paths are proxied."
        },
        live_side_effects_allowed: false
      },
      { status: 404 }
    );
  }

  const apiBaseUrl = configuredApiBaseUrl();
  if (!apiBaseUrl) {
    return unavailable(`${localDemoApiBaseUrlEnvName} is not configured for the staff-web runtime.`);
  }

  try {
    const upstreamUrl = new URL(`${apiBaseUrl}/`);
    upstreamUrl.pathname = `${upstreamUrl.pathname.replace(/\/$/, "")}/${path}`;
    upstreamUrl.search = request.nextUrl.search;
    const upstreamHeaders = new Headers({ accept: "application/json" });
    const requestId = request.headers.get("x-request-id");
    const correlationId = request.headers.get("x-correlation-id");
    if (requestId) upstreamHeaders.set("x-request-id", requestId);
    if (correlationId) upstreamHeaders.set("x-correlation-id", correlationId);
    const upstream = await fetch(upstreamUrl, {
      headers: upstreamHeaders,
      cache: "no-store"
    });
    const contentType = upstream.headers.get("content-type") ?? "application/json";
    const body = await upstream.text();

    return new NextResponse(body, {
      status: upstream.status,
      headers: {
        "content-type": contentType,
        "cache-control": "no-store"
      }
    });
  } catch {
    return unavailable("Local demo API proxy is unavailable; retry after the sample API is configured.");
  }
}

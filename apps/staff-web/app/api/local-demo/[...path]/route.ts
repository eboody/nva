import { NextRequest, NextResponse } from "next/server";
import { fetchLocalDemoApi, safeLocalDemoApiPath } from "./local-demo-proxy.mjs";

const localDemoApiBaseUrlEnvName = "PET_RESORT_API_BASE_URL";

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

async function proxyLocalDemoApi(
  request: NextRequest,
  context: { params: Promise<{ path?: string[] }> },
  method: "GET" | "POST"
) {
  const params = await context.params;
  const path = safeLocalDemoApiPath(params.path ?? []);

  if (!path) {
    return NextResponse.json(
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

  const apiBaseUrl = configuredApiBaseUrl();
  if (!apiBaseUrl) {
    return unavailable(`${localDemoApiBaseUrlEnvName} is not configured for the staff-web runtime.`);
  }

  try {
    const upstreamHeaders = new Headers({ accept: "application/json" });
    const requestId = request.headers.get("x-request-id");
    const correlationId = request.headers.get("x-correlation-id");
    if (requestId) upstreamHeaders.set("x-request-id", requestId);
    if (correlationId) upstreamHeaders.set("x-correlation-id", correlationId);
    const upstream = await fetchLocalDemoApi({
      apiBaseUrl,
      segments: params.path ?? [],
      search: request.nextUrl.search,
      method,
      headers: upstreamHeaders
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

export async function GET(
  request: NextRequest,
  context: { params: Promise<{ path?: string[] }> }
) {
  return proxyLocalDemoApi(request, context, "GET");
}

export async function POST(
  request: NextRequest,
  context: { params: Promise<{ path?: string[] }> }
) {
  return proxyLocalDemoApi(request, context, "POST");
}

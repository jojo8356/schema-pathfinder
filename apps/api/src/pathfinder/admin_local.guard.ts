import { CanActivate, ExecutionContext, Injectable } from "@nestjs/common";
import { isAdminLocalRequest } from "./admin_local_guard.mjs";

interface RequestLike {
  headers?: Record<string, string | string[] | undefined>;
  hostname?: string;
  ip?: string;
  socketRemoteAddress?: string;
}

@Injectable()
export class AdminLocalGuard implements CanActivate {
  canActivate(context: ExecutionContext): boolean {
    const request = context.switchToHttp().getRequest<RequestLike>();
    return isAdminLocalRequest(request);
  }
}

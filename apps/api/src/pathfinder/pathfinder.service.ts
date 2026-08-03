import { Injectable } from "@nestjs/common";
import { createPathfinderService } from "./pathfinder_service.mjs";
import { PathRequestDto } from "./dto/path_request.dto";

@Injectable()
export class PathfinderService {
  private readonly service = createPathfinderService();

  findPath(request: PathRequestDto): Promise<unknown> {
    return this.service.findPath(request);
  }
}

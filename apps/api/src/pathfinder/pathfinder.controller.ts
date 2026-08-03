import { Body, Controller, Post } from "@nestjs/common";
import { PathRequestDto } from "./dto/path_request.dto";
import { PathfinderService } from "./pathfinder.service";

@Controller("admin/pathfinder")
export class PathfinderController {
  constructor(private readonly pathfinderService: PathfinderService) {}

  @Post("path")
  findPath(@Body() request: PathRequestDto): Promise<unknown> {
    return this.pathfinderService.findPath(request);
  }
}

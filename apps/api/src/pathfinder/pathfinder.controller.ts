import { Body, Controller, Post, UseGuards } from "@nestjs/common";
import { AdminLocalGuard } from "./admin_local.guard";
import { PathRequestDto } from "./dto/path_request.dto";
import { PathfinderService } from "./pathfinder.service";

@Controller("admin/pathfinder")
@UseGuards(AdminLocalGuard)
export class PathfinderController {
  constructor(private readonly pathfinderService: PathfinderService) {}

  @Post("path")
  findPath(@Body() request: PathRequestDto): Promise<unknown> {
    return this.pathfinderService.findPath(request);
  }
}

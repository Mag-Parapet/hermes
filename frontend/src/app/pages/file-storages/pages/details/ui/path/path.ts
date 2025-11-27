import { Component, EventEmitter, input, Output, signal } from '@angular/core';

@Component({
  selector: 'path',
  imports: [],
  templateUrl: './path.html',
  styleUrl: './path.css',
})
export class Path {
  path = input<string[]>([]);
  @Output() pathChange = new EventEmitter();
  currentPath = signal<string[]>([]);

  ngOnChanges() {
    this.currentPath.set(this.path());
  }

  onPathSegmentClick(path: string[], index: number) {
    if (index == -1) {
      this.pathChange.emit([]);
    } else {
      const newPath = path.slice(0, index + 1);
      this.currentPath.set(newPath);
      this.pathChange.emit(newPath);
    }
  }
}

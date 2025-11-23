import { Component, computed, EventEmitter, input, Output, signal, Signal, WritableSignal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'content-table',
  imports: [RouterLink, FormsModule],
  templateUrl: './content-table.html',
  styleUrl: './content-table.scss',
})
export class ContentTable {
  columns: any = input([]);
  data: any = input(); 

  @Output() pageChange = new EventEmitter<number>();
  @Output() pageSizeChange = new EventEmitter<number>();

  pagesRange: Signal<number[]> = computed(() => {
    const totalPages = this.data()?.pagination?.totalPages || 1;
    return Array.from({ length: totalPages }, (_, i) => i + 1);
  });

  page: WritableSignal<number> = signal(1);

  onChangePage(page: string) {
    this.page.set(Number(page));
    this.pageChange.emit(this.page());
  }

  onChangePageSize(size: string) {
    this.pageSizeChange.emit(Number(size));
  }

  onPreviousPage() {
    if (this.page() > 1) {
      this.page.set(this.page() - 1);
      this.pageChange.emit(this.page());
    }
  }

  onNextPage() {
    const totalPages = this.data()?.pagination?.totalPages || 1;
    if (this.page() < totalPages) {
      this.page.set(this.page() + 1);
      this.pageChange.emit(this.page());
    }
  }
}

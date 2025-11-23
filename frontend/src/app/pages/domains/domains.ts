import { AsyncPipe, DatePipe } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormBuilder, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { ContentTable } from '@core/components/content-table/content-table';
import { DomainsService } from '@core/services/domains';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import { map } from 'rxjs';

@Component({
  selector: 'app-domains',
  imports: [ContentTable, RouterLink, AsyncPipe, ReactiveFormsModule],
  templateUrl: './domains.html',
  styleUrl: './domains.css',
  providers: [DatePipe]
})
export class Domains {
  domainsService = inject(DomainsService);
  datePipe = inject(DatePipe);
  store = inject(Store);

  fb = new FormBuilder();
  form = this.fb.group({
    search: [''],
    port: []
  });

  columns = ['Domain', 'Port', 'SSL', 'Active', 'Last Updated'];

  data = [];

  page = signal<number>(1);
  pageSize = signal<number>(10);

  domains$: any

  constructor() {
    this.setDomains()
    this.form.valueChanges.pipe(takeUntilDestroyed()).subscribe(() => {
      this.page.set(1);
      this.setDomains();
    })
  }

  onPageChange(page: number) {
    this.page.set(page);
    this.setDomains();
  }

  onPageSizeChange(size: number) {
    this.pageSize.set(size);
    this.setDomains();
  }

  setDomains() {
    const { search, port } = this.form.value;
    this.store.dispatch(setLoading({ state: true }));
    this.domains$ = this.domainsService.getAllDomains(
      this.page(),
      this.pageSize(),
      search || '',
      port || 0,
    ).pipe(
      map((res: any) => {
        this.store.dispatch(setLoading({ state: false }));
        console.log(res)
        return {
          rows: res.data.domains.map((item: any) => [
            item.domain,
            item.port,
            item.isSsl ? 'Yes' : 'No',
            item.isActive ? 'Yes' : 'No',
            this.datePipe.transform(item.updatedAt, 'medium'),
            item.id
          ]),
          pagination: res.data.pagination
        }
      }
    ))
  }
}
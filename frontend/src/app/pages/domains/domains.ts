import { AsyncPipe, DatePipe } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormBuilder, ReactiveFormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { ContentTable } from '@core/components/content-table/content-table';
import { UnitConvertPipe } from '@core/pipes/unit-convert-pipe';
import { DomainsService } from '@core/services/domains';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import Snackbar from 'awesome-snackbar';
import { catchError, map } from 'rxjs';

@Component({
  selector: 'app-domains',
  imports: [ContentTable, RouterLink, AsyncPipe, ReactiveFormsModule],
  templateUrl: './domains.html',
  styleUrl: './domains.css',
  providers: [DatePipe, UnitConvertPipe]
})
export class Domains {
  domainsService = inject(DomainsService);
  datePipe = inject(DatePipe);
  store = inject(Store);
  unitConvertPipe = inject(UnitConvertPipe);

  fb = new FormBuilder();
  form = this.fb.group({
    search: [''],
    domainType: [''],
    isActive: [''],
  });

  columns = ['Domain', 'Type', 'Max Body', 'SSL', 'Active', 'Last Updated'];

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
    const { search, domainType, isActive } = this.form.value;
    this.store.dispatch(setLoading({ state: true }));
    this.domains$ = this.domainsService.getAllDomains(
      this.page(),
      this.pageSize(),
      search || '',
      domainType || '',
      isActive || ''
    ).pipe(
      map((res: any) => {
        this.store.dispatch(setLoading({ state: false }));
        return {
          rows: res.data.domains.map((item: any) => [
            item.domain,
            item.domainType,
            this.unitConvertPipe.transform(item.clientMaxBodySize ?? 'N/A'),
            item.isSsl ? 'Yes' : 'No',
            item.isActive ? 'Yes' : 'No',
            this.datePipe.transform(item.updatedAt, 'medium'),
            item.id
          ]),
          pagination: res.data.pagination
        }
      },
      catchError((err) => {
        new Snackbar(err.error?.message || `Failed to load domains`, {
          iconSrc: '/error.png',
          position: 'bottom-right',
        });
        this.store.dispatch(setLoading({ state: false }));
        return [];
      })
    ))
  }
}
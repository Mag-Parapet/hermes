import { AsyncPipe, DatePipe } from '@angular/common';
import { Component, inject } from '@angular/core';
import { ActivatedRoute, Router, RouterLink } from '@angular/router';
import { ConfirmDialog } from '@core/components/confirm-dialog/confirm-dialog';
import { DomainsService } from '@core/services/domains';
import { addParamHeader } from '@core/utils/add-param-header';
import { DialogService } from '@ngneat/dialog';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import { catchError, tap } from 'rxjs';

@Component({
  selector: 'app-details',
  imports: [AsyncPipe, DatePipe, RouterLink],
  templateUrl: './details.html',
  styleUrl: './details.css',
})
export class Details {
  route = inject(ActivatedRoute);
  router = inject(Router);
  domainsService = inject(DomainsService);
  dialog = inject(DialogService)
  store = inject(Store)

  domainId = this.route.snapshot.paramMap.get('id')!;

  domain$ = this.domainsService.getDomainById(this.domainId).pipe(
    tap(() => this.store.dispatch(setLoading({ state: false }))),
    addParamHeader(':domainId', 'data.domain'),
    catchError((error: any) => {
      if (error.status === 404) {
        this.router.navigate(['/404']);
      }
      this.router.navigate(['/domains']);
      return [];
    })
  )

  constructor() {
    this.store.dispatch(setLoading({ state: true }));
  }

  onDeleteDomain() {
    let d = this.dialog.open(ConfirmDialog, {
      data: {
        title: 'Confirm Deletion',
        content: 'Are you sure you want to delete this domain?'
      }
    });
    d.afterClosed$.subscribe((result: any) => {
      if (result?.confirm) {
        this.store.dispatch(setLoading({ state: true }));
        this.domainsService.deleteDomain(this.domainId).subscribe(
          () => {
            this.store.dispatch(setLoading({ state: false }));
            this.router.navigate(['/domains']);
          }
        );
      }
    });
  }
}

import { AsyncPipe } from '@angular/common';
import { Component, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { DomainsService } from '@core/services/domains';
import { addParamHeader } from '@core/utils/add-param-header';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';
import { catchError, tap } from 'rxjs';

@Component({
  selector: 'app-edit',
  imports: [ReactiveFormsModule, AsyncPipe],
  templateUrl: './edit.html',
  styleUrl: './edit.css',
})
export class Edit {
  domainsService = inject(DomainsService);
  router = inject(Router);
  route = inject(ActivatedRoute);
  store = inject(Store);

  domainId = this.route.snapshot.paramMap.get('id')!;

  domain$ = this.domainsService.getDomainById(this.domainId || '').pipe(
    tap((res: any) => {
      this.store.dispatch(setLoading({ state: false }))
      this.form.patchValue({
        domain: res.data.domain,
        port: res.data.port,
        maxBodySize: res.data.maxBodySize,
        isSsl: res.data.isSsl,
        isActive: res.data.isActive,
        sslCertificatePath: res.data.sslCertificatePath || '',
        sslCertificateKeyPath: res.data.sslCertificateKeyPath || ''
      })
      this.isLoading.set(false);
    }),
    addParamHeader(':domainId', 'data.domain'),
    catchError((error: any) => {
      if (error.status === 404) {
        this.router.navigate(['/404/']);
      }
      this.router.navigate(['/domains/']);
      return [];
    })
  )

  fb = new FormBuilder();
  form = this.fb.group({
    domain: ['', [Validators.required]],
    port: [3000, [Validators.required, Validators.min(1), Validators.max(65535)]],
    maxBodySize: [1, [Validators.required, Validators.min(1)]],
    isSsl: [true],
    isActive: [true],
    sslCertificatePath: [''],
    sslCertificateKeyPath: [''],
  })

  isLoading = signal(true);

  onSubmit() {
    if (this.form.valid) {
      this.store.dispatch(setLoading({ state: true }));
      const { domain, port, maxBodySize, isSsl, isActive, sslCertificatePath, sslCertificateKeyPath } = this.form.value;
      this.isLoading.set(true);
      this.domainsService.updateDomain(this.domainId, {
        domain: domain ?? '',
        port: port ?? 3000,
        maxBodySize: maxBodySize ?? 1,
        isSsl: isSsl ?? true,
        isActive: isActive ?? true,
        sslCertificatePath: sslCertificatePath ?? '',
        sslCertificateKeyPath: sslCertificateKeyPath ?? '',
      }).subscribe(
        (res: any) => {
          this.isLoading.set(false);
          this.store.dispatch(setLoading({ state: false }));
          this.router.navigate(['/domains/', res.data.id]);
        }
      );
    }
  }

  isInvalid(controlName: string): boolean {
    const control = this.form.get(controlName);
    return !!(
      control &&
      control.invalid &&
      control.touched
    );
  }
}
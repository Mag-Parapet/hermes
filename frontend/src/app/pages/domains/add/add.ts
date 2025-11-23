import { Component, inject, signal } from '@angular/core';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router } from '@angular/router';
import { DomainsService } from '@core/services/domains';
import { Store } from '@ngrx/store';
import { setLoading } from 'app/state/loading/loading.actions';

@Component({
  selector: 'app-add',
  imports: [ReactiveFormsModule],
  templateUrl: './add.html',
  styleUrl: './add.css',
})
export class Add {
  domainsService = inject(DomainsService);
  router = inject(Router);
  store = inject(Store);

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

  isLoading = signal(false);

  onSubmit() {
    if (this.form.valid) {
      this.store.dispatch(setLoading({ state: true }));
      const { domain, port, maxBodySize, isSsl, isActive, sslCertificatePath, sslCertificateKeyPath } = this.form.value;
      this.isLoading.set(true);
      this.domainsService.createDomain({
        domain: domain ?? '',
        port: port ?? 3000,
        maxBodySize: maxBodySize ?? 1,
        isSsl: isSsl ?? true,
        isActive: isActive ?? true,
        sslCertificatePath: sslCertificatePath ?? '',
        sslCertificateKeyPath: sslCertificateKeyPath ?? ''
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
